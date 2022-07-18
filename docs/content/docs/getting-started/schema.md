+++
title = "Schema"
description = "One page summary of how to start a new AdiDoks project."
date = 2021-05-01T08:20:00+00:00
updated = 2021-05-01T08:20:00+00:00
draft = false
weight = 20
sort_by = "weight"
template = "docs/page.html"
+++

<details>
  <summary><strong>Table of Contents</strong></summary>

  * [Query](#query)
  * [Mutation](#mutation)
  * [Objects](#objects)
    * [Amendment](#amendment)
    * [ArgumentResult](#argumentresult)
    * [AuthTokenResult](#authtokenresult)
    * [BallotMeasureResult](#ballotmeasureresult)
    * [Bill](#bill)
    * [BillResult](#billresult)
    * [BillResultConnection](#billresultconnection)
    * [BillResultEdge](#billresultedge)
    * [Calendar](#calendar)
    * [Candidate](#candidate)
    * [DeleteArgumentResult](#deleteargumentresult)
    * [DeleteBallotMeasureResult](#deleteballotmeasureresult)
    * [DeleteBillResult](#deletebillresult)
    * [DeleteElectionResult](#deleteelectionresult)
    * [DeleteIssueTagResult](#deleteissuetagresult)
    * [DeleteOfficeResult](#deleteofficeresult)
    * [DeleteOrganizationResult](#deleteorganizationresult)
    * [DeletePoliticianResult](#deletepoliticianresult)
    * [DeleteRaceResult](#deleteraceresult)
    * [DeleteVotingGuideResult](#deletevotingguideresult)
    * [ElectionResult](#electionresult)
    * [Endorsements](#endorsements)
    * [GeneralInfo](#generalinfo)
    * [GetCandidateBioResponse](#getcandidatebioresponse)
    * [History](#history)
    * [IssueTagResult](#issuetagresult)
    * [Office](#office)
    * [OfficeResult](#officeresult)
    * [OrganizationResult](#organizationresult)
    * [OrganizationResultConnection](#organizationresultconnection)
    * [OrganizationResultEdge](#organizationresultedge)
    * [PageInfo](#pageinfo)
    * [PasswordEntropyResult](#passwordentropyresult)
    * [PoliticianResult](#politicianresult)
    * [PoliticianResultConnection](#politicianresultconnection)
    * [PoliticianResultEdge](#politicianresultedge)
    * [Progress](#progress)
    * [RaceCandidateResult](#racecandidateresult)
    * [RaceResult](#raceresult)
    * [RaceResultsResult](#raceresultsresult)
    * [RatingResult](#ratingresult)
    * [RatingResultConnection](#ratingresultconnection)
    * [RatingResultEdge](#ratingresultedge)
    * [Referral](#referral)
    * [RollCall](#rollcall)
    * [Sast](#sast)
    * [Session](#session)
    * [Sponsor](#sponsor)
    * [Subject](#subject)
    * [Supplement](#supplement)
    * [Text](#text)
    * [UserResult](#userresult)
    * [Vote](#vote)
    * [VotingGuideCandidateResult](#votingguidecandidateresult)
    * [VotingGuideResult](#votingguideresult)
    * [VsRating](#vsrating)
  * [Inputs](#inputs)
    * [BallotMeasureSearch](#ballotmeasuresearch)
    * [BillSearch](#billsearch)
    * [CreateArgumentInput](#createargumentinput)
    * [CreateBallotMeasureInput](#createballotmeasureinput)
    * [CreateBillInput](#createbillinput)
    * [CreateElectionInput](#createelectioninput)
    * [CreateIssueTagInput](#createissuetaginput)
    * [CreateOfficeInput](#createofficeinput)
    * [CreateOrConnectIssueTagInput](#createorconnectissuetaginput)
    * [CreateOrConnectOrganizationInput](#createorconnectorganizationinput)
    * [CreateOrConnectPoliticianInput](#createorconnectpoliticianinput)
    * [CreateOrganizationInput](#createorganizationinput)
    * [CreatePoliticianInput](#createpoliticianinput)
    * [CreateRaceInput](#createraceinput)
    * [ElectionSearchInput](#electionsearchinput)
    * [IssueTagSearch](#issuetagsearch)
    * [OfficeSearch](#officesearch)
    * [OrganizationSearch](#organizationsearch)
    * [PoliticianFilter](#politicianfilter)
    * [PoliticianSearch](#politiciansearch)
    * [RaceSearch](#racesearch)
    * [UpdateArgumentInput](#updateargumentinput)
    * [UpdateBallotMeasureInput](#updateballotmeasureinput)
    * [UpdateBillInput](#updatebillinput)
    * [UpdateElectionInput](#updateelectioninput)
    * [UpdateIssueTagInput](#updateissuetaginput)
    * [UpdateOfficeInput](#updateofficeinput)
    * [UpdateOrganizationInput](#updateorganizationinput)
    * [UpdatePoliticianInput](#updatepoliticianinput)
    * [UpdateRaceInput](#updateraceinput)
    * [UpsertVotingGuideCandidateInput](#upsertvotingguidecandidateinput)
    * [UpsertVotingGuideInput](#upsertvotingguideinput)
  * [Enums](#enums)
    * [ArgumentPosition](#argumentposition)
    * [AuthorType](#authortype)
    * [Chamber](#chamber)
    * [Chambers](#chambers)
    * [District](#district)
    * [ElectionScope](#electionscope)
    * [LegislationStatus](#legislationstatus)
    * [PoliticalParty](#politicalparty)
    * [PoliticalScope](#politicalscope)
    * [RaceType](#racetype)
    * [Role](#role)
    * [State](#state)
  * [Scalars](#scalars)
    * [Boolean](#boolean)
    * [Float](#float)
    * [ID](#id)
    * [Int](#int)
    * [JSON](#json)
    * [NaiveDate](#naivedate)
    * [String](#string)
    * [UUID](#uuid)
  * [Unions](#unions)
    * [AuthorResult](#authorresult)

</details>

## Query
<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>userCount</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td>

Get all users

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>allBallotMeasures</strong></td>
<td valign="top">[<a href="#ballotmeasureresult">BallotMeasureResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotMeasures</strong></td>
<td valign="top">[<a href="#ballotmeasureresult">BallotMeasureResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">search</td>
<td valign="top"><a href="#ballotmeasuresearch">BallotMeasureSearch</a>!</td>
<td>

Search by voteStatus, name, or slug

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>bills</strong></td>
<td valign="top"><a href="#billresultconnection">BillResultConnection</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">search</td>
<td valign="top"><a href="#billsearch">BillSearch</a>!</td>
<td>

Search by voteStatus, title, or slug

</td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">after</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">before</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">first</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">last</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>billBySlug</strong></td>
<td valign="top"><a href="#billresult">BillResult</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">slug</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>elections</strong></td>
<td valign="top">[<a href="#electionresult">ElectionResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">search</td>
<td valign="top"><a href="#electionsearchinput">ElectionSearchInput</a>!</td>
<td>

Search by slug or title

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>nextElection</strong></td>
<td valign="top"><a href="#electionresult">ElectionResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>upcomingElections</strong></td>
<td valign="top">[<a href="#electionresult">ElectionResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionById</strong></td>
<td valign="top"><a href="#electionresult">ElectionResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionBySlug</strong></td>
<td valign="top"><a href="#electionresult">ElectionResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">slug</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>issueTagBySlug</strong></td>
<td valign="top"><a href="#issuetagresult">IssueTagResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">slug</td>
<td valign="top"><a href="#string">String</a>!</td>
<td>

Search issue tag by slug

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>allIssueTags</strong></td>
<td valign="top">[<a href="#issuetagresult">IssueTagResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>issueTags</strong></td>
<td valign="top">[<a href="#issuetagresult">IssueTagResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">search</td>
<td valign="top"><a href="#issuetagsearch">IssueTagSearch</a>!</td>
<td>

Search by issue tag name

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>health</strong></td>
<td valign="top"><a href="#boolean">Boolean</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>offices</strong></td>
<td valign="top">[<a href="#officeresult">OfficeResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">search</td>
<td valign="top"><a href="#officesearch">OfficeSearch</a></td>
<td>

Search by office title or state

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeById</strong></td>
<td valign="top"><a href="#officeresult">OfficeResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeBySlug</strong></td>
<td valign="top"><a href="#officeresult">OfficeResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">slug</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>organizations</strong></td>
<td valign="top"><a href="#organizationresultconnection">OrganizationResultConnection</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">search</td>
<td valign="top"><a href="#organizationsearch">OrganizationSearch</a></td>
<td>

Search by organization name

</td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">after</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">before</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">first</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">last</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>organizationBySlug</strong></td>
<td valign="top"><a href="#organizationresult">OrganizationResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">slug</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicianById</strong></td>
<td valign="top"><a href="#politicianresult">PoliticianResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicianBySlug</strong></td>
<td valign="top"><a href="#politicianresult">PoliticianResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">slug</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicians</strong></td>
<td valign="top"><a href="#politicianresultconnection">PoliticianResultConnection</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">search</td>
<td valign="top"><a href="#politiciansearch">PoliticianSearch</a></td>
<td>

Search by homeState or lastName

</td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">filter</td>
<td valign="top"><a href="#politicianfilter">PoliticianFilter</a></td>
<td>

Filter by politicalScope or chambers

</td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">after</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">before</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">first</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">last</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>races</strong></td>
<td valign="top">[<a href="#raceresult">RaceResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">search</td>
<td valign="top"><a href="#racesearch">RaceSearch</a></td>
<td>

Search by race title or state

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceById</strong></td>
<td valign="top"><a href="#raceresult">RaceResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceBySlug</strong></td>
<td valign="top"><a href="#raceresult">RaceResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">slug</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>validateEmailAvailable</strong></td>
<td valign="top"><a href="#boolean">Boolean</a>!</td>
<td>

Validate that a user does not already exist with this email

</td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">email</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>validatePasswordEntropy</strong></td>
<td valign="top"><a href="#passwordentropyresult">PasswordEntropyResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">password</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>currentUser</strong></td>
<td valign="top"><a href="#authtokenresult">AuthTokenResult</a></td>
<td>

Provides current user based on JWT found in client's access_token cookie

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votingGuidesByIds</strong></td>
<td valign="top">[<a href="#votingguideresult">VotingGuideResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">ids</td>
<td valign="top">[<a href="#id">ID</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votingGuideById</strong></td>
<td valign="top"><a href="#votingguideresult">VotingGuideResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td>

Voting guide id

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votingGuidesByUserId</strong></td>
<td valign="top">[<a href="#votingguideresult">VotingGuideResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">userId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td>

User id

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionVotingGuideByUserId</strong></td>
<td valign="top"><a href="#votingguideresult">VotingGuideResult</a></td>
<td>

Returns a single voting guide for the given election and user

</td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">electionId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td>

Election ID

</td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">userId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td>

User ID

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>userProfile</strong></td>
<td valign="top"><a href="#userresult">UserResult</a>!</td>
<td>

Publicly accessible user information

</td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">userId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
</tbody>
</table>

## Mutation
<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>updateArgument</strong></td>
<td valign="top"><a href="#argumentresult">ArgumentResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#updateargumentinput">UpdateArgumentInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteArgument</strong></td>
<td valign="top"><a href="#deleteargumentresult">DeleteArgumentResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>upvoteArgument</strong></td>
<td valign="top"><a href="#boolean">Boolean</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">argumentId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">populistUserId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>downvoteArgument</strong></td>
<td valign="top"><a href="#boolean">Boolean</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">argumentId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">populistUserId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>createPolitician</strong></td>
<td valign="top"><a href="#politicianresult">PoliticianResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#createpoliticianinput">CreatePoliticianInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>updatePolitician</strong></td>
<td valign="top"><a href="#politicianresult">PoliticianResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#updatepoliticianinput">UpdatePoliticianInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deletePolitician</strong></td>
<td valign="top"><a href="#deletepoliticianresult">DeletePoliticianResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>createOrganization</strong></td>
<td valign="top"><a href="#organizationresult">OrganizationResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#createorganizationinput">CreateOrganizationInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>updateOrganization</strong></td>
<td valign="top"><a href="#organizationresult">OrganizationResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#updateorganizationinput">UpdateOrganizationInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteOrganization</strong></td>
<td valign="top"><a href="#deleteorganizationresult">DeleteOrganizationResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>createBill</strong></td>
<td valign="top"><a href="#billresult">BillResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#createbillinput">CreateBillInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>updateBill</strong></td>
<td valign="top"><a href="#billresult">BillResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">legiscanBillId</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#updatebillinput">UpdateBillInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteBill</strong></td>
<td valign="top"><a href="#deletebillresult">DeleteBillResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>createBallotMeasure</strong></td>
<td valign="top"><a href="#ballotmeasureresult">BallotMeasureResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">electionId</td>
<td valign="top"><a href="#uuid">UUID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#createballotmeasureinput">CreateBallotMeasureInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>updateBallotMeasure</strong></td>
<td valign="top"><a href="#ballotmeasureresult">BallotMeasureResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#updateballotmeasureinput">UpdateBallotMeasureInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteBallotMeasure</strong></td>
<td valign="top"><a href="#deleteballotmeasureresult">DeleteBallotMeasureResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>createElection</strong></td>
<td valign="top"><a href="#electionresult">ElectionResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#createelectioninput">CreateElectionInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>updateElection</strong></td>
<td valign="top"><a href="#electionresult">ElectionResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#updateelectioninput">UpdateElectionInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteElection</strong></td>
<td valign="top"><a href="#deleteelectionresult">DeleteElectionResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>createIssueTag</strong></td>
<td valign="top"><a href="#issuetagresult">IssueTagResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#createissuetaginput">CreateIssueTagInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>updateIssueTag</strong></td>
<td valign="top"><a href="#issuetagresult">IssueTagResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#updateissuetaginput">UpdateIssueTagInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteIssueTag</strong></td>
<td valign="top"><a href="#deleteissuetagresult">DeleteIssueTagResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>createOffice</strong></td>
<td valign="top"><a href="#officeresult">OfficeResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#createofficeinput">CreateOfficeInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>updateOffice</strong></td>
<td valign="top"><a href="#officeresult">OfficeResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#updateofficeinput">UpdateOfficeInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteOffice</strong></td>
<td valign="top"><a href="#deleteofficeresult">DeleteOfficeResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>createRace</strong></td>
<td valign="top"><a href="#raceresult">RaceResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#createraceinput">CreateRaceInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>updateRace</strong></td>
<td valign="top"><a href="#raceresult">RaceResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#updateraceinput">UpdateRaceInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteRace</strong></td>
<td valign="top"><a href="#deleteraceresult">DeleteRaceResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>upsertVotingGuide</strong></td>
<td valign="top"><a href="#votingguideresult">VotingGuideResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#upsertvotingguideinput">UpsertVotingGuideInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>upsertVotingGuideCandidate</strong></td>
<td valign="top"><a href="#votingguidecandidateresult">VotingGuideCandidateResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">input</td>
<td valign="top"><a href="#upsertvotingguidecandidateinput">UpsertVotingGuideCandidateInput</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteVotingGuide</strong></td>
<td valign="top"><a href="#deletevotingguideresult">DeleteVotingGuideResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">id</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>deleteVotingGuideCandidateNote</strong></td>
<td valign="top"><a href="#votingguidecandidateresult">VotingGuideCandidateResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">votingGuideId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">candidateId</td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
</tbody>
</table>

## Objects

### Amendment

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>amendmentId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>adopted</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamberId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>date</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>mime</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>mimeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>url</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>stateLink</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### ArgumentResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>authorId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>authorType</strong></td>
<td valign="top"><a href="#authortype">AuthorType</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>position</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>body</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>author</strong></td>
<td valign="top"><a href="#authorresult">AuthorResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votes</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### AuthTokenResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>username</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>email</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>role</strong></td>
<td valign="top"><a href="#role">Role</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>userProfile</strong></td>
<td valign="top"><a href="#userresult">UserResult</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### BallotMeasureResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legislationStatus</strong></td>
<td valign="top"><a href="#legislationstatus">LegislationStatus</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotState</strong></td>
<td valign="top"><a href="#state">State</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotMeasureCode</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>measureType</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>definitions</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officialSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>populistSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>fullTextUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>arguments</strong></td>
<td valign="top">[<a href="#ballotmeasureresult">BallotMeasureResult</a>!]!</td>
<td></td>
</tr>
</tbody>
</table>

### Bill

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>billId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>changeHash</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sessionId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>session</strong></td>
<td valign="top"><a href="#session">Session</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>url</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>stateLink</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>completed</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>status</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>statusDate</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>progress</strong></td>
<td valign="top">[<a href="#progress">Progress</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>state</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>stateId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>billNumber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>billType</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>billTypeId</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>body</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>bodyId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>currentBody</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>currentBodyId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>committee</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>referrals</strong></td>
<td valign="top">[<a href="#referral">Referral</a>!]</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>pendingCommitteeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>history</strong></td>
<td valign="top">[<a href="#history">History</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sponsors</strong></td>
<td valign="top">[<a href="#sponsor">Sponsor</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sasts</strong></td>
<td valign="top">[<a href="#sast">Sast</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>subjects</strong></td>
<td valign="top">[<a href="#subject">Subject</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>texts</strong></td>
<td valign="top">[<a href="#text">Text</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votes</strong></td>
<td valign="top">[<a href="#vote">Vote</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>amendments</strong></td>
<td valign="top">[<a href="#amendment">Amendment</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>supplements</strong></td>
<td valign="top">[<a href="#supplement">Supplement</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>calendar</strong></td>
<td valign="top">[<a href="#calendar">Calendar</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>statusType</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### BillResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>billNumber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legislationStatus</strong></td>
<td valign="top"><a href="#legislationstatus">LegislationStatus</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officialSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>populistSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>fullTextUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legiscanBillId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>history</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>arguments</strong></td>
<td valign="top">[<a href="#argumentresult">ArgumentResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legiscanData</strong></td>
<td valign="top"><a href="#bill">Bill</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### BillResultConnection

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>pageInfo</strong></td>
<td valign="top"><a href="#pageinfo">PageInfo</a>!</td>
<td>

Information to aid in pagination.

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>edges</strong></td>
<td valign="top">[<a href="#billresultedge">BillResultEdge</a>]</td>
<td>

A list of edges.

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>totalCount</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td>

Total result set count

</td>
</tr>
</tbody>
</table>

### BillResultEdge

An edge in a connection.

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>node</strong></td>
<td valign="top"><a href="#billresult">BillResult</a>!</td>
<td>

The item at the end of the edge

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>cursor</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td>

A cursor for use in pagination

</td>
</tr>
</tbody>
</table>

### Calendar

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>typeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>typeField</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>date</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>time</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>location</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### Candidate

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>candidateId</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>crpId</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>photo</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>firstName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>nickName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>middleName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>preferredName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>lastName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>suffix</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>birthDate</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>birthPlace</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>pronunciation</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>gender</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>family</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>homeCity</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>homeState</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>education</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>profession</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>political</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>congMembership</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>orgMembership</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>religion</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>specialMsg</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeleteArgumentResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeleteBallotMeasureResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeleteBillResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeleteElectionResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeleteIssueTagResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeleteOfficeResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeleteOrganizationResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeletePoliticianResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeleteRaceResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### DeleteVotingGuideResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### ElectionResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionDate</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>races</strong></td>
<td valign="top">[<a href="#raceresult">RaceResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>racesByUserDistricts</strong></td>
<td valign="top">[<a href="#raceresult">RaceResult</a>!]!</td>
<td>

Show races relevant to the user based on their address

</td>
</tr>
</tbody>
</table>

### Endorsements

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>politicians</strong></td>
<td valign="top">[<a href="#politicianresult">PoliticianResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>organizations</strong></td>
<td valign="top">[<a href="#organizationresult">OrganizationResult</a>!]!</td>
<td></td>
</tr>
</tbody>
</table>

### GeneralInfo

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>linkBack</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### GetCandidateBioResponse

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>generalInfo</strong></td>
<td valign="top"><a href="#generalinfo">GeneralInfo</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>candidate</strong></td>
<td valign="top"><a href="#candidate">Candidate</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>office</strong></td>
<td valign="top"><a href="#office">Office</a></td>
<td></td>
</tr>
</tbody>
</table>

### History

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>date</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>action</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamberId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>importance</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### IssueTagResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicians</strong></td>
<td valign="top">[<a href="#politicianresult">PoliticianResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>organizations</strong></td>
<td valign="top">[<a href="#organizationresult">OrganizationResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>bills</strong></td>
<td valign="top">[<a href="#billresult">BillResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotMeasures</strong></td>
<td valign="top">[<a href="#ballotmeasureresult">BallotMeasureResult</a>!]!</td>
<td></td>
</tr>
</tbody>
</table>

### Office

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top">[<a href="#string">String</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>typeField</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>status</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>parties</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>stateId</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>termEnd</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>district</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>lastElect</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>nextElect</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>termStart</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>districtId</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>firstElect</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>shortTitle</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### OfficeResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeType</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>district</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>districtType</strong></td>
<td valign="top"><a href="#district">District</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicalScope</strong></td>
<td valign="top"><a href="#politicalscope">PoliticalScope</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionScope</strong></td>
<td valign="top"><a href="#electionscope">ElectionScope</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>state</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>municipality</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>incumbent</strong></td>
<td valign="top"><a href="#politicianresult">PoliticianResult</a></td>
<td></td>
</tr>
</tbody>
</table>

### OrganizationResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>thumbnailImageUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>websiteUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>issueTags</strong></td>
<td valign="top">[<a href="#issuetagresult">IssueTagResult</a>!]!</td>
<td></td>
</tr>
</tbody>
</table>

### OrganizationResultConnection

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>pageInfo</strong></td>
<td valign="top"><a href="#pageinfo">PageInfo</a>!</td>
<td>

Information to aid in pagination.

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>edges</strong></td>
<td valign="top">[<a href="#organizationresultedge">OrganizationResultEdge</a>]</td>
<td>

A list of edges.

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>totalCount</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td>

Total result set count

</td>
</tr>
</tbody>
</table>

### OrganizationResultEdge

An edge in a connection.

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>node</strong></td>
<td valign="top"><a href="#organizationresult">OrganizationResult</a>!</td>
<td>

The item at the end of the edge

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>cursor</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td>

A cursor for use in pagination

</td>
</tr>
</tbody>
</table>

### PageInfo

Information about pagination in a connection

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>hasPreviousPage</strong></td>
<td valign="top"><a href="#boolean">Boolean</a>!</td>
<td>

When paginating backwards, are there more items?

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>hasNextPage</strong></td>
<td valign="top"><a href="#boolean">Boolean</a>!</td>
<td>

When paginating forwards, are there more items?

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>startCursor</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td>

When paginating backwards, the cursor to continue.

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>endCursor</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td>

When paginating forwards, the cursor to continue.

</td>
</tr>
</tbody>
</table>

### PasswordEntropyResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>valid</strong></td>
<td valign="top"><a href="#boolean">Boolean</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>score</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>message</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### PoliticianResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>firstName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>middleName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>lastName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>suffix</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>preferredName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>biography</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>biographySource</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>homeState</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>dateOfBirth</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeId</strong></td>
<td valign="top"><a href="#id">ID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>thumbnailImageUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>websiteUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>campaignWebsiteUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>facebookUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>twitterUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>instagramUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>youtubeUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>linkedinUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>tiktokUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>email</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>party</strong></td>
<td valign="top"><a href="#politicalparty">PoliticalParty</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartCandidateId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartCandidateBio</strong></td>
<td valign="top"><a href="#getcandidatebioresponse">GetCandidateBioResponse</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartCandidateRatings</strong></td>
<td valign="top">[<a href="#vsrating">VsRating</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>upcomingRaceId</strong></td>
<td valign="top"><a href="#id">ID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceWins</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceLosses</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>fullName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>age</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ratings</strong></td>
<td valign="top"><a href="#ratingresultconnection">RatingResultConnection</a>!</td>
<td>

Leverages Votesmart ratings data for the time being

</td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">after</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">before</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">first</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">last</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>yearsInPublicOffice</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td>

Calculates the total years a politician has been in office using
the votesmart politicial experience array.  Does not take into account
objects where the politician is considered a 'candidate'

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>endorsements</strong></td>
<td valign="top"><a href="#endorsements">Endorsements</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>issueTags</strong></td>
<td valign="top">[<a href="#issuetagresult">IssueTagResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sponsoredBills</strong></td>
<td valign="top"><a href="#billresultconnection">BillResultConnection</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">after</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">before</td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">first</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">last</td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>currentOffice</strong></td>
<td valign="top"><a href="#officeresult">OfficeResult</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>upcomingRace</strong></td>
<td valign="top"><a href="#raceresult">RaceResult</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votes</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" align="right" valign="top">raceId</td>
<td valign="top"><a href="#uuid">UUID</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### PoliticianResultConnection

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>pageInfo</strong></td>
<td valign="top"><a href="#pageinfo">PageInfo</a>!</td>
<td>

Information to aid in pagination.

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>edges</strong></td>
<td valign="top">[<a href="#politicianresultedge">PoliticianResultEdge</a>]</td>
<td>

A list of edges.

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>totalCount</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td>

Total result set count

</td>
</tr>
</tbody>
</table>

### PoliticianResultEdge

An edge in a connection.

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>node</strong></td>
<td valign="top"><a href="#politicianresult">PoliticianResult</a>!</td>
<td>

The item at the end of the edge

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>cursor</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td>

A cursor for use in pagination

</td>
</tr>
</tbody>
</table>

### Progress

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>date</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>event</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### RaceCandidateResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>candidateId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votes</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votePercentage</strong></td>
<td valign="top"><a href="#float">Float</a></td>
<td></td>
</tr>
</tbody>
</table>

### RaceResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceType</strong></td>
<td valign="top"><a href="#racetype">RaceType</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>party</strong></td>
<td valign="top"><a href="#politicalparty">PoliticalParty</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>state</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotpediaLink</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>earlyVotingBeginsDate</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officialWebsite</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionId</strong></td>
<td valign="top"><a href="#id">ID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>office</strong></td>
<td valign="top"><a href="#officeresult">OfficeResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>candidates</strong></td>
<td valign="top">[<a href="#politicianresult">PoliticianResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>results</strong></td>
<td valign="top"><a href="#raceresultsresult">RaceResultsResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionDate</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a></td>
<td></td>
</tr>
</tbody>
</table>

### RaceResultsResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>votesByCandidate</strong></td>
<td valign="top">[<a href="#racecandidateresult">RaceCandidateResult</a>!]!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>totalVotes</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>winner</strong></td>
<td valign="top"><a href="#politicianresult">PoliticianResult</a></td>
<td></td>
</tr>
</tbody>
</table>

### RatingResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>vsRating</strong></td>
<td valign="top"><a href="#vsrating">VsRating</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>organization</strong></td>
<td valign="top"><a href="#organizationresult">OrganizationResult</a></td>
<td></td>
</tr>
</tbody>
</table>

### RatingResultConnection

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>pageInfo</strong></td>
<td valign="top"><a href="#pageinfo">PageInfo</a>!</td>
<td>

Information to aid in pagination.

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>edges</strong></td>
<td valign="top">[<a href="#ratingresultedge">RatingResultEdge</a>]</td>
<td>

A list of edges.

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>totalCount</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td>

Total result set count

</td>
</tr>
</tbody>
</table>

### RatingResultEdge

An edge in a connection.

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>node</strong></td>
<td valign="top"><a href="#ratingresult">RatingResult</a>!</td>
<td>

The item at the end of the edge

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>cursor</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td>

A cursor for use in pagination

</td>
</tr>
</tbody>
</table>

### Referral

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>date</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>committeeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamberId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### RollCall

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>rollCallId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>billId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>date</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>desc</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>yea</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>nay</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>nv</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>absent</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>total</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>passed</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamberId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votes</strong></td>
<td valign="top">[<a href="#vote">Vote</a>!]!</td>
<td></td>
</tr>
</tbody>
</table>

### Sast

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>typeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>typeField</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sastBillNumber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sastBillId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### Session

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>sessionId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sessionName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sessionTitle</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>yearStart</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>yearEnd</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>special</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### Sponsor

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>peopleId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>personHash</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>partyId</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>party</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>roleId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>role</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>firstName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>middleName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>lastName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>suffix</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>nickname</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>district</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ftmEid</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>opensecretsId</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotpedia</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sponsorTypeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sponsorOrder</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>committeeSponsor</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>committeeId</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### Subject

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>subjectId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>subjectName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### Supplement

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>supplementId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>date</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>typeField</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>typeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>mime</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>mimeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>url</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>stateLink</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### Text

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>docId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>date</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>typeField</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>typeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>mime</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>mimeId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>url</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>stateLink</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>textSize</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### UserResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>username</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>email</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>firstName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>lastName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>profilePictureUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### Vote

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>rollCallId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>rollCallData</strong></td>
<td valign="top"><a href="#rollcall">RollCall</a></td>
<td>

This field is not returned from get_bill, but can be populated with a subsequent call to `get_roll_call`

</td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>date</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>desc</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>yea</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>nay</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>nv</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>absent</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>total</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>passed</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamberId</strong></td>
<td valign="top"><a href="#int">Int</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>url</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>stateLink</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### VotingGuideCandidateResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>candidateId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>isEndorsement</strong></td>
<td valign="top"><a href="#boolean">Boolean</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>note</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politician</strong></td>
<td valign="top"><a href="#politicianresult">PoliticianResult</a>!</td>
<td></td>
</tr>
</tbody>
</table>

### VotingGuideResult

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>userId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>user</strong></td>
<td valign="top"><a href="#userresult">UserResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>election</strong></td>
<td valign="top"><a href="#electionresult">ElectionResult</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>candidates</strong></td>
<td valign="top">[<a href="#votingguidecandidateresult">VotingGuideCandidateResult</a>!]!</td>
<td></td>
</tr>
</tbody>
</table>

### VsRating

<table>
<thead>
<tr>
<th align="left">Field</th>
<th align="right">Argument</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>categories</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>rating</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ratingId</strong></td>
<td valign="top"><a href="#json">JSON</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ratingName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ratingText</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>sigId</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>timespan</strong></td>
<td valign="top"><a href="#json">JSON</a>!</td>
<td></td>
</tr>
</tbody>
</table>

## Inputs

### BallotMeasureSearch

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotState</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legislationStatus</strong></td>
<td valign="top"><a href="#legislationstatus">LegislationStatus</a></td>
<td></td>
</tr>
</tbody>
</table>

### BillSearch

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>billNumber</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legislationStatus</strong></td>
<td valign="top"><a href="#legislationstatus">LegislationStatus</a></td>
<td></td>
</tr>
</tbody>
</table>

### CreateArgumentInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>authorId</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>position</strong></td>
<td valign="top"><a href="#argumentposition">ArgumentPosition</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>body</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### CreateBallotMeasureInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legislationStatus</strong></td>
<td valign="top"><a href="#legislationstatus">LegislationStatus</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotState</strong></td>
<td valign="top"><a href="#state">State</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotMeasureCode</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>measureType</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>definitions</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officialSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>populistSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>fullTextUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### CreateBillInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>billNumber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legislationStatus</strong></td>
<td valign="top"><a href="#legislationstatus">LegislationStatus</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officialSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>populistSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>fullTextUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legiscanBillId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legiscanData</strong></td>
<td valign="top"><a href="#json">JSON</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartBillId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>arguments</strong></td>
<td valign="top">[<a href="#createargumentinput">CreateArgumentInput</a>!]</td>
<td></td>
</tr>
</tbody>
</table>

### CreateElectionInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionDate</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a>!</td>
<td>

Must use format YYYY-MM-DD

</td>
</tr>
</tbody>
</table>

### CreateIssueTagInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>category</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### CreateOfficeInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeType</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>district</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>districtType</strong></td>
<td valign="top"><a href="#district">District</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamber</strong></td>
<td valign="top"><a href="#chamber">Chamber</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionScope</strong></td>
<td valign="top"><a href="#electionscope">ElectionScope</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicalScope</strong></td>
<td valign="top"><a href="#politicalscope">PoliticalScope</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>state</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>municipality</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>incumbentId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>termLength</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
</tbody>
</table>

### CreateOrConnectIssueTagInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>create</strong></td>
<td valign="top">[<a href="#createissuetaginput">CreateIssueTagInput</a>!]</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>connect</strong></td>
<td valign="top">[<a href="#string">String</a>!]</td>
<td></td>
</tr>
</tbody>
</table>

### CreateOrConnectOrganizationInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>create</strong></td>
<td valign="top">[<a href="#createorganizationinput">CreateOrganizationInput</a>!]</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>connect</strong></td>
<td valign="top">[<a href="#string">String</a>!]</td>
<td></td>
</tr>
</tbody>
</table>

### CreateOrConnectPoliticianInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>create</strong></td>
<td valign="top">[<a href="#createpoliticianinput">CreatePoliticianInput</a>!]</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>connect</strong></td>
<td valign="top">[<a href="#string">String</a>!]</td>
<td></td>
</tr>
</tbody>
</table>

### CreateOrganizationInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>thumbnailImageUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>websiteUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>facebookUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>twitterUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>instagramUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>email</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartSigId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>headquartersAddressId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>headquartersPhone</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>taxClassification</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>issueTags</strong></td>
<td valign="top"><a href="#createorconnectissuetaginput">CreateOrConnectIssueTagInput</a></td>
<td></td>
</tr>
</tbody>
</table>

### CreatePoliticianInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>firstName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>middleName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>lastName</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>suffix</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>preferredName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>biography</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>biographySource</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>homeState</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>dateOfBirth</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>thumbnailImageUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>websiteUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>campaignWebsiteUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>facebookUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>twitterUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>instagramUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>youtubeUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>linkedinUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>tiktokUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>email</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>party</strong></td>
<td valign="top"><a href="#politicalparty">PoliticalParty</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>issueTags</strong></td>
<td valign="top"><a href="#createorconnectissuetaginput">CreateOrConnectIssueTagInput</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>organizationEndorsements</strong></td>
<td valign="top"><a href="#createorconnectorganizationinput">CreateOrConnectOrganizationInput</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicianEndorsements</strong></td>
<td valign="top"><a href="#createorconnectpoliticianinput">CreateOrConnectPoliticianInput</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartCandidateId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartCandidateBio</strong></td>
<td valign="top"><a href="#json">JSON</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartCandidateRatings</strong></td>
<td valign="top"><a href="#json">JSON</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legiscanPeopleId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>crpCandidateId</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>fecCandidateId</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>upcomingRaceId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceWins</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceLosses</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
</tbody>
</table>

### CreateRaceInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeId</strong></td>
<td valign="top"><a href="#uuid">UUID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceType</strong></td>
<td valign="top"><a href="#racetype">RaceType</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>party</strong></td>
<td valign="top"><a href="#politicalparty">PoliticalParty</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>state</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotpediaLink</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>earlyVotingBeginsDate</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officialWebsite</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>winnerId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
</tbody>
</table>

### ElectionSearchInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### IssueTagSearch

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### OfficeSearch

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>query</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>state</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicalScope</strong></td>
<td valign="top"><a href="#politicalscope">PoliticalScope</a></td>
<td></td>
</tr>
</tbody>
</table>

### OrganizationSearch

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### PoliticianFilter

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>politicalScope</strong></td>
<td valign="top"><a href="#politicalscope">PoliticalScope</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chambers</strong></td>
<td valign="top"><a href="#chambers">Chambers</a></td>
<td></td>
</tr>
</tbody>
</table>

### PoliticianSearch

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>homeState</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>party</strong></td>
<td valign="top"><a href="#politicalparty">PoliticalParty</a></td>
<td></td>
</tr>
</tbody>
</table>

### RaceSearch

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>query</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>state</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
</tbody>
</table>

### UpdateArgumentInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>position</strong></td>
<td valign="top"><a href="#argumentposition">ArgumentPosition</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>body</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### UpdateBallotMeasureInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legislationStatus</strong></td>
<td valign="top"><a href="#legislationstatus">LegislationStatus</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotState</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotMeasureCode</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>measureType</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>definitions</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officialSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>populistSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>fullTextUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### UpdateBillInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>billNumber</strong></td>
<td valign="top"><a href="#string">String</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legislationStatus</strong></td>
<td valign="top"><a href="#legislationstatus">LegislationStatus</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officialSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>populistSummary</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>fullTextUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legiscanBillId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legiscanData</strong></td>
<td valign="top"><a href="#json">JSON</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>arguments</strong></td>
<td valign="top">[<a href="#createargumentinput">CreateArgumentInput</a>!]</td>
<td></td>
</tr>
</tbody>
</table>

### UpdateElectionInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionDate</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a></td>
<td>

Must use format YYYY-MM-DD

</td>
</tr>
</tbody>
</table>

### UpdateIssueTagInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>category</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### UpdateOfficeInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeType</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>district</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>districtType</strong></td>
<td valign="top"><a href="#district">District</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>chamber</strong></td>
<td valign="top"><a href="#chamber">Chamber</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionScope</strong></td>
<td valign="top"><a href="#electionscope">ElectionScope</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicalScope</strong></td>
<td valign="top"><a href="#politicalscope">PoliticalScope</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>state</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>municipality</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>incumbentId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>termLength</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
</tbody>
</table>

### UpdateOrganizationInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>name</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>thumbnailImageUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>websiteUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>facebookUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>twitterUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>instagramUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>email</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartSigId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>headquartersAddressId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>headquartersPhone</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>taxClassification</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>issueTags</strong></td>
<td valign="top"><a href="#createorconnectissuetaginput">CreateOrConnectIssueTagInput</a></td>
<td></td>
</tr>
</tbody>
</table>

### UpdatePoliticianInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>firstName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>middleName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>lastName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>suffix</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>preferredName</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>biography</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>biographySource</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>homeState</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>dateOfBirth</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>thumbnailImageUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>websiteUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>campaignWebsiteUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>facebookUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>twitterUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>instagramUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>youtubeUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>linkedinUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>tiktokUrl</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>email</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>party</strong></td>
<td valign="top"><a href="#politicalparty">PoliticalParty</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>issueTags</strong></td>
<td valign="top"><a href="#createorconnectissuetaginput">CreateOrConnectIssueTagInput</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>organizationEndorsements</strong></td>
<td valign="top"><a href="#createorconnectorganizationinput">CreateOrConnectOrganizationInput</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>politicianEndorsements</strong></td>
<td valign="top"><a href="#createorconnectpoliticianinput">CreateOrConnectPoliticianInput</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartCandidateId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartCandidateBio</strong></td>
<td valign="top"><a href="#json">JSON</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>votesmartCandidateRatings</strong></td>
<td valign="top"><a href="#json">JSON</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>legiscanPeopleId</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>crpCandidateId</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>fecCandidateId</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>upcomingRaceId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceWins</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceLosses</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
</tbody>
</table>

### UpdateRaceInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>slug</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officeId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>raceType</strong></td>
<td valign="top"><a href="#racetype">RaceType</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>party</strong></td>
<td valign="top"><a href="#politicalparty">PoliticalParty</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>ballotpediaLink</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>earlyVotingBeginsDate</strong></td>
<td valign="top"><a href="#naivedate">NaiveDate</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>officialWebsite</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>state</strong></td>
<td valign="top"><a href="#state">State</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>winnerId</strong></td>
<td valign="top"><a href="#uuid">UUID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>totalVotes</strong></td>
<td valign="top"><a href="#int">Int</a></td>
<td></td>
</tr>
</tbody>
</table>

### UpsertVotingGuideCandidateInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>votingGuideId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>candidateId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>isEndorsement</strong></td>
<td valign="top"><a href="#boolean">Boolean</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>note</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

### UpsertVotingGuideInput

<table>
<thead>
<tr>
<th colspan="2" align="left">Field</th>
<th align="left">Type</th>
<th align="left">Description</th>
</tr>
</thead>
<tbody>
<tr>
<td colspan="2" valign="top"><strong>id</strong></td>
<td valign="top"><a href="#id">ID</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>electionId</strong></td>
<td valign="top"><a href="#id">ID</a>!</td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>title</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
<tr>
<td colspan="2" valign="top"><strong>description</strong></td>
<td valign="top"><a href="#string">String</a></td>
<td></td>
</tr>
</tbody>
</table>

## Enums

### ArgumentPosition

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>SUPPORT</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>NEUTRAL</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>OPPOSE</strong></td>
<td></td>
</tr>
</tbody>
</table>

### AuthorType

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>POLITICIAN</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>ORGANIZATION</strong></td>
<td></td>
</tr>
</tbody>
</table>

### Chamber

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>HOUSE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>SENATE</strong></td>
<td></td>
</tr>
</tbody>
</table>

### Chambers

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>ALL</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>HOUSE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>SENATE</strong></td>
<td></td>
</tr>
</tbody>
</table>

### District

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>US_CONGRESSIONAL</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>STATE_SENATE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>STATE_HOUSE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>SCHOOL</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>CITY</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>COUNTY</strong></td>
<td></td>
</tr>
</tbody>
</table>

### ElectionScope

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>NATIONAL</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>STATE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>COUNTY</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>CITY</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>DISTRICT</strong></td>
<td></td>
</tr>
</tbody>
</table>

### LegislationStatus

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>INTRODUCED</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>PASSED_HOUSE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>PASSED_SENATE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>FAILED_HOUSE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>FAILED_SENATE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>RESOLVING_DIFFERENCES</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>SENT_TO_EXECUTIVE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>BECAME_LAW</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>VETOED</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>UNKNOWN</strong></td>
<td></td>
</tr>
</tbody>
</table>

### PoliticalParty

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>DEMOCRATIC</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>REPUBLICAN</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>LIBERTARIAN</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>FREEDOM</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>UNITY</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>GREEN</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>CONSTITUTION</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>INDEPENDENT</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>APPROVAL_VOTING</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>UNAFFILIATED</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>UNKNOWN</strong></td>
<td></td>
</tr>
</tbody>
</table>

### PoliticalScope

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>LOCAL</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>STATE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>FEDERAL</strong></td>
<td></td>
</tr>
</tbody>
</table>

### RaceType

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>PRIMARY</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>GENERAL</strong></td>
<td></td>
</tr>
</tbody>
</table>

### Role

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>SUPERUSER</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>STAFF</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>PREMIUM</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>BASIC</strong></td>
<td></td>
</tr>
</tbody>
</table>

### State

<table>
<thead>
<th align="left">Value</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong>AL</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>AK</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>AS</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>AZ</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>AR</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>CA</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>CO</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>CT</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>DC</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>FM</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>DE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>FL</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>GA</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>GU</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>HI</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>ID</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>IL</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>IN</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>IA</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>KS</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>KY</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>LA</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>ME</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>MH</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>MD</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>MA</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>MI</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>MN</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>MS</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>MO</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>MT</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>NE</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>NV</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>NH</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>NJ</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>NM</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>NY</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>NC</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>ND</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>MP</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>OH</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>OK</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>OR</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>PW</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>PA</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>PR</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>RI</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>SC</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>SD</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>TN</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>TX</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>UT</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>VT</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>VI</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>VA</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>WA</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>WV</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>WI</strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong>WY</strong></td>
<td></td>
</tr>
</tbody>
</table>

## Scalars

### Boolean

The `Boolean` scalar type represents `true` or `false`.

### Float

The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).

### ID

### Int

The `Int` scalar type represents non-fractional whole numeric values.

### JSON

A scalar that can represent any JSON value.

### NaiveDate

ISO 8601 calendar date without timezone.
Format: %Y-%m-%d

# Examples

* `1994-11-13`
* `2000-02-24`

### String

The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.

### UUID

A UUID is a unique 128-bit number, stored as 16 octets. UUIDs are parsed as Strings
within GraphQL. UUIDs are used to assign unique identifiers to entities without requiring a central
allocating authority.

# References

* [Wikipedia: Universally Unique Identifier](http://en.wikipedia.org/wiki/Universally_unique_identifier)
* [RFC4122: A Universally Unique IDentifier (UUID) URN Namespace](http://tools.ietf.org/html/rfc4122)


## Unions

### AuthorResult

<table>
<thead>
<th align="left">Type</th>
<th align="left">Description</th>
</thead>
<tbody>
<tr>
<td valign="top"><strong><a href="#politicianresult">PoliticianResult</a></strong></td>
<td></td>
</tr>
<tr>
<td valign="top"><strong><a href="#organizationresult">OrganizationResult</a></strong></td>
<td></td>
</tr>
</tbody>
</table>
